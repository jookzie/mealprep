// Package errorx holds structured error types shared across layers.
package errorx

import (
	"fmt"

	"github.com/google/uuid"
)

type ErrNotFound struct {
	Resource string
	ID       uuid.UUID
	Internal error
}

func NotFound(resource string, id uuid.UUID, internal error) ErrNotFound {
	return ErrNotFound{Resource: resource, ID: id, Internal: internal}
}

func (e ErrNotFound) Error() string {
	return fmt.Sprintf("%s not found with id = '%s': %v", e.Resource, e.ID, e.Internal)
}

func (e ErrNotFound) Unwrap() error {
	return e.Internal
}
