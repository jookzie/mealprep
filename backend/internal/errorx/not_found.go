package errorx

import (
	"fmt"

	"github.com/google/uuid"
)

// ErrNotFound reports that a resource with a given id does not exist.
type ErrNotFound struct {
	Resource string
	ID       uuid.UUID
	Internal error
}

// NotFound builds an ErrNotFound for the given resource and id.
func NotFound(resource string, id uuid.UUID, internal error) ErrNotFound {
	return ErrNotFound{Resource: resource, ID: id, Internal: internal}
}

func (e ErrNotFound) Error() string {
	return fmt.Sprintf("%s not found with id = '%s': %v", e.Resource, e.ID, e.Internal)
}

// Unwrap exposes the internal error to errors.Is / errors.As.
func (e ErrNotFound) Unwrap() error {
	return e.Internal
}
