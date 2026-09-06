package meal

import (
	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/errorx"
)

func (s *Service) Get(id uuid.UUID) (Meal, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	for _, m := range s.meals {
		if m.ID == id {
			return m, nil
		}
	}

	return Meal{}, errorx.NotFound("meal", id, nil)
}
