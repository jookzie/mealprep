package calendar

import (
	"context"
	"time"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

// Assign sets the day plan for a date, replacing whatever was there.
func (r Repository) Assign(ctx context.Context, date time.Time, planID uuid.UUID) (domain.CalendarDay, error) {
	now := sqliterepo.Now()
	row, err := r.queries.AssignCalendarDay(ctx, db.AssignCalendarDayParams{
		Date:      date.Format(dateLayout),
		DayPlanID: planID.String(),
		CreatedAt: now,
		UpdatedAt: now,
	})
	if err != nil {
		return domain.CalendarDay{}, err
	}
	return fromRow(row)
}
