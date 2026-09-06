package calendar

import (
	"context"
	"time"

	"github.com/mertan/mealprep/internal/domain"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) List(ctx context.Context, from, to time.Time) ([]domain.CalendarDay, error) {
	rows, err := r.queries.ListCalendarDays(ctx, db.ListCalendarDaysParams{
		Date:   from.Format(dateLayout),
		Date_2: to.Format(dateLayout),
	})
	if err != nil {
		return nil, err
	}

	days := make([]domain.CalendarDay, 0, len(rows))
	for _, row := range rows {
		day, err := fromRow(row)
		if err != nil {
			return nil, err
		}
		days = append(days, day)
	}
	return days, nil
}
