package meal

import mealservice "github.com/mertan/mealprep/internal/service/meal"

func New(meals *mealservice.Service) Handler {
	return Handler{meals: meals}
}
