package dayplan

import (
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

var ErrInvalidID = errors.New("invalid id in day plan rows")

func fromRow(row db.DayPlan, meals []db.DayPlanMeal) (domain.DayPlan, error) {
	id, err := uuid.Parse(row.ID)
	if err != nil {
		return domain.DayPlan{}, errors.Join(ErrInvalidID, err)
	}
	createdAt, err := sqliterepo.ParseTime(row.CreatedAt)
	if err != nil {
		return domain.DayPlan{}, err
	}
	updatedAt, err := sqliterepo.ParseTime(row.UpdatedAt)
	if err != nil {
		return domain.DayPlan{}, err
	}
	deletedAt, err := sqliterepo.ParseNullTime(row.DeletedAt)
	if err != nil {
		return domain.DayPlan{}, err
	}

	plan := domain.DayPlan{
		ID:        id,
		Label:     row.Label,
		MealIDs:   make([]uuid.UUID, 0, len(meals)),
		CreatedAt: createdAt,
		UpdatedAt: updatedAt,
		DeletedAt: deletedAt,
	}
	for _, meal := range meals {
		mealID, err := uuid.Parse(meal.MealID)
		if err != nil {
			return domain.DayPlan{}, errors.Join(ErrInvalidID, err)
		}
		plan.MealIDs = append(plan.MealIDs, mealID)
	}

	return plan, nil
}

func fromRows(rows []db.DayPlan, meals []db.DayPlanMeal) ([]domain.DayPlan, error) {
	byPlan := make(map[string][]db.DayPlanMeal, len(rows))
	for _, meal := range meals {
		byPlan[meal.DayPlanID] = append(byPlan[meal.DayPlanID], meal)
	}

	plans := make([]domain.DayPlan, 0, len(rows))
	for _, row := range rows {
		plan, err := fromRow(row, byPlan[row.ID])
		if err != nil {
			return nil, err
		}
		plans = append(plans, plan)
	}
	return plans, nil
}
