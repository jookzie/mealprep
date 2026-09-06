// Package errorx holds structured error types shared across layers.
package errorx

import "fmt"

type ErrNotFound struct {
	Resource string
	ID       string
	Internal error
}

func NotFound(resource, id string, internal error) ErrNotFound {
	return ErrNotFound{Resource: resource, ID: id, Internal: internal}
}

func (e ErrNotFound) Error() string {
	if e.Internal == nil {
		return fmt.Sprintf("%s not found with id = '%s'", e.Resource, e.ID)
	}
	return fmt.Sprintf("%s not found with id = '%s': %v", e.Resource, e.ID, e.Internal)
}

func (e ErrNotFound) Unwrap() error {
	return e.Internal
}
