package targets

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

func (s Service) Get(ctx context.Context) (domain.Targets, error) {
	return s.repository.Get(ctx)
}
