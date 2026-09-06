// Package targets holds the single user's daily targets.
package targets

import (
	"context"

	"github.com/mertan/mealprep/internal/domain"
)

type Repository interface {
	Get(ctx context.Context) (domain.Targets, error)
	Set(ctx context.Context, macros domain.Macros) (domain.Targets, error)
}

type Service struct {
	repository Repository
}
