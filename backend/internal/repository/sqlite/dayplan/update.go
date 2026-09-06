package dayplan

import (
	"context"
	"database/sql"
	"errors"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

// Update replaces the label and the whole meal list; old links are soft-deleted.
func (r Repository) Update(ctx context.Context, plan domain.DayPlan) (domain.DayPlan, error) {
	now := sqliterepo.Now()

	err := sqliterepo.Transact(ctx, r.conn, func(queries *db.Queries) error {
		_, err := queries.UpdateDayPlan(ctx, db.UpdateDayPlanParams{
			Label:     plan.Label,
			UpdatedAt: now,
			ID:        plan.ID.String(),
		})
		if errors.Is(err, sql.ErrNoRows) {
			return errorx.NotFound("day plan", plan.ID.String(), err)
		}
		if err != nil {
			return err
		}

		err = queries.SoftDeleteDayPlanMeals(ctx, db.SoftDeleteDayPlanMealsParams{
			DeletedAt: sql.NullString{String: now, Valid: true},
			UpdatedAt: now,
			DayPlanID: plan.ID.String(),
		})
		if err != nil {
			return err
		}

		return insertMeals(ctx, queries, plan, now)
	})
	if err != nil {
		return domain.DayPlan{}, err
	}

	return r.Get(ctx, plan.ID)
}
