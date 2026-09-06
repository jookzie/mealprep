package meal

import (
	"errors"

	"github.com/google/uuid"
)

var ErrInvalidMeal = errors.New("invalid meal")

func (s *Service) Create(label string, calories int) (Meal, error) {
	if label == "" {
		return Meal{}, errors.Join(ErrInvalidMeal, errors.New("label is empty"))
	}
	if calories <= 0 {
		return Meal{}, errors.Join(ErrInvalidMeal, errors.New("calories must be positive"))
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	created := Meal{ID: uuid.New(), Label: label, Calories: calories}
	s.meals = append(s.meals, created)

	return created, nil
}
