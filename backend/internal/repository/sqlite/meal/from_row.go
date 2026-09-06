package meal

import (
	"errors"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

var ErrInvalidID = errors.New("invalid id in meal rows")

func fromRow(row db.Meal, servings []db.MealServing) (domain.Meal, error) {
	id, err := uuid.Parse(row.ID)
	if err != nil {
		return domain.Meal{}, errors.Join(ErrInvalidID, err)
	}
	createdAt, err := sqliterepo.ParseTime(row.CreatedAt)
	if err != nil {
		return domain.Meal{}, err
	}
	updatedAt, err := sqliterepo.ParseTime(row.UpdatedAt)
	if err != nil {
		return domain.Meal{}, err
	}
	deletedAt, err := sqliterepo.ParseNullTime(row.DeletedAt)
	if err != nil {
		return domain.Meal{}, err
	}

	meal := domain.Meal{
		ID:        id,
		Label:     row.Label,
		Servings:  make([]domain.Serving, 0, len(servings)),
		CreatedAt: createdAt,
		UpdatedAt: updatedAt,
		DeletedAt: deletedAt,
	}
	for _, serving := range servings {
		productID, err := uuid.Parse(serving.ProductID)
		if err != nil {
			return domain.Meal{}, errors.Join(ErrInvalidID, err)
		}
		meal.Servings = append(meal.Servings, domain.Serving{ProductID: productID, Amount: serving.Amount})
	}

	return meal, nil
}

// fromRows pairs each meal with its servings; both inputs come ordered from the queries.
func fromRows(rows []db.Meal, servings []db.MealServing) ([]domain.Meal, error) {
	byMeal := make(map[string][]db.MealServing, len(rows))
	for _, serving := range servings {
		byMeal[serving.MealID] = append(byMeal[serving.MealID], serving)
	}

	meals := make([]domain.Meal, 0, len(rows))
	for _, row := range rows {
		meal, err := fromRow(row, byMeal[row.ID])
		if err != nil {
			return nil, err
		}
		meals = append(meals, meal)
	}
	return meals, nil
}
