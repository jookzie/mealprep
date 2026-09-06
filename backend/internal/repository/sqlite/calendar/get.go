package calendar

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/errorx"
)

func (r Repository) Get(ctx context.Context, date time.Time) (domain.CalendarDay, error) {
	key := date.Format(dateLayout)
	row, err := r.queries.GetCalendarDay(ctx, key)
	if errors.Is(err, sql.ErrNoRows) {
		return domain.CalendarDay{}, errorx.NotFound("calendar day", key, err)
	}
	if err != nil {
		return domain.CalendarDay{}, err
	}
	return fromRow(row)
}
