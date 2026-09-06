package calendar

import (
	"errors"
	"time"

	"github.com/google/uuid"

	"github.com/mertan/mealprep/internal/domain"
	sqliterepo "github.com/mertan/mealprep/internal/repository/sqlite"
	"github.com/mertan/mealprep/internal/repository/sqlite/db"
)

var (
	ErrInvalidID   = errors.New("invalid day plan id in calendar rows")
	ErrInvalidDate = errors.New("invalid date in calendar rows")
)

func fromRow(row db.CalendarDay) (domain.CalendarDay, error) {
	date, err := time.Parse(dateLayout, row.Date)
	if err != nil {
		return domain.CalendarDay{}, errors.Join(ErrInvalidDate, err)
	}
	planID, err := uuid.Parse(row.DayPlanID)
	if err != nil {
		return domain.CalendarDay{}, errors.Join(ErrInvalidID, err)
	}
	createdAt, err := sqliterepo.ParseTime(row.CreatedAt)
	if err != nil {
		return domain.CalendarDay{}, err
	}
	updatedAt, err := sqliterepo.ParseTime(row.UpdatedAt)
	if err != nil {
		return domain.CalendarDay{}, err
	}
	deletedAt, err := sqliterepo.ParseNullTime(row.DeletedAt)
	if err != nil {
		return domain.CalendarDay{}, err
	}

	return domain.CalendarDay{
		Date:      date,
		DayPlanID: planID,
		CreatedAt: createdAt,
		UpdatedAt: updatedAt,
		DeletedAt: deletedAt,
	}, nil
}
