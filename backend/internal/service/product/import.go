package product

import (
	"context"
	"errors"
	"fmt"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrIncompleteProduct = errors.New("catalog product lacks one of the four macros")

// Import snapshots a catalog entry into the user's products. The snapshot is
// never refreshed from the catalog afterwards.
func (s Service) Import(ctx context.Context, code string) (domain.Product, error) {
	entry, err := s.catalog.Get(ctx, code)
	if err != nil {
		return domain.Product{}, catalogError(err)
	}
	if !entry.Complete {
		return domain.Product{}, errors.Join(ErrIncompleteProduct, fmt.Errorf("code %s", code))
	}

	product := domain.Product{
		ID:         uuid.New(),
		Name:       entry.Name,
		Unit:       entry.Unit,
		Macros:     entry.Macros,
		Nutrients:  entry.Nutrients,
		SourceCode: entry.Code,
	}
	if err = validate(product); err != nil {
		return domain.Product{}, err
	}

	return s.repository.Create(ctx, product)
}
