package product

import (
	"context"
	"errors"
	"strings"

	"github.com/mertan/mealprep/internal/domain"
)

var ErrEmptyQuery = errors.New("search query is empty")

func (s Service) Search(ctx context.Context, query string) ([]domain.CatalogEntry, error) {
	query = strings.TrimSpace(query)
	if query == "" {
		return nil, ErrEmptyQuery
	}
	entries, err := s.catalog.Search(ctx, query)
	if err != nil {
		return nil, catalogError(err)
	}
	return entries, nil
}
