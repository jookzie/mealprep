package meal

import (
	"context"

	"github.com/google/uuid"
)

func (s Service) Delete(ctx context.Context, id uuid.UUID) error {
	return s.repository.Delete(ctx, id)
}
