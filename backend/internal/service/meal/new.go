package meal

import "github.com/google/uuid"

func New() *Service {
	return &Service{meals: []Meal{
		{ID: uuid.MustParse("11111111-1111-1111-1111-111111111111"), Label: "Porridge", Calories: 350},
		{ID: uuid.MustParse("22222222-2222-2222-2222-222222222222"), Label: "Chicken and rice", Calories: 720},
	}}
}
