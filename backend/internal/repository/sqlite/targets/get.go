package targets

import (
	"context"
	"database/sql"
	"errors"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
)

func (r Repository) Get(ctx context.Context) (domain.Targets, error) {
	row, err := r.queries.GetTargets(ctx)
	if errors.Is(err, sql.ErrNoRows) {
		return domain.Targets{}, errorx.NotFound("targets", "1", err)
	}
	if err != nil {
		return domain.Targets{}, err
	}
	return fromRow(row)
}
