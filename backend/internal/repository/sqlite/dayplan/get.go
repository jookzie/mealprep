package dayplan

import (
	"context"
	"database/sql"
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
)

func (r Repository) Get(ctx context.Context, id uuid.UUID) (domain.DayPlan, error) {
	row, err := r.queries.GetDayPlan(ctx, id.String())
	if errors.Is(err, sql.ErrNoRows) {
		return domain.DayPlan{}, errorx.NotFound("day plan", id.String(), err)
	}
	if err != nil {
		return domain.DayPlan{}, err
	}

	meals, err := r.queries.ListDayPlanMealsByDayPlanIDs(ctx, []string{row.ID})
	if err != nil {
		return domain.DayPlan{}, err
	}

	return fromRow(row, meals)
}
