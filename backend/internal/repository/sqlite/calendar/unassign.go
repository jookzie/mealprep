package calendar

import (
	"context"
	"time"

	"github.com/mertan/mealprep/internal/errorx"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

func (r Repository) Unassign(ctx context.Context, date time.Time) error {
	key := date.Format(dateLayout)
	affected, err := r.queries.SoftDeleteCalendarDay(ctx, db.SoftDeleteCalendarDayParams{
		DeletedAt: sqliterepo.NullNow(),
		UpdatedAt: sqliterepo.Now(),
		Date:      key,
	})
	if err != nil {
		return err
	}
	if affected == 0 {
		return errorx.NotFound("calendar day", key, nil)
	}
	return nil
}
