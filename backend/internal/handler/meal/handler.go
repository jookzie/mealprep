// Package meal serves the meal endpoints of the generated API.
package meal

import mealservice "github.com/mertan/mealprep/internal/service/meal"

type Handler struct {
	meals *mealservice.Service
}
