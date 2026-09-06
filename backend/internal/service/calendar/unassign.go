package calendar

import (
	"context"
	"time"
)

func (s Service) Unassign(ctx context.Context, date time.Time) error {
	return s.repository.Unassign(ctx, date)
}
