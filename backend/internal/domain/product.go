package domain

import (
	"time"

	"github.com/google/uuid"
)

// Unit is what a product's nutrients are measured per 100 of, and what its
// servings are measured in.
type Unit string

const (
	Gram       Unit = "g"
	Millilitre Unit = "ml"
)

type Product struct {
	ID     uuid.UUID
	Name   string
	Unit   Unit
	Macros Macros
	// Nutrients holds every further value per 100 Unit, keyed by name.
	Nutrients map[string]float64
	// Brand is who makes it. Two imports often share a Name and differ only
	// here, so it is part of the product's identity; empty when unknown.
	Brand string
	// SourceCode is the Open Food Facts code a snapshot came from; empty when
	// the user created the product.
	SourceCode string
	CreatedAt  time.Time
	UpdatedAt  time.Time
	DeletedAt  *time.Time
}

// CatalogEntry is a product as an external catalog offers it, before import.
// Complete is false when the four macros are not all present.
type CatalogEntry struct {
	Code      string
	Name      string
	Unit      Unit
	Macros    Macros
	Nutrients map[string]float64
	Brand     string
	// ImageURL points at the catalog's own copy. The images are licensed
	// separately from the data, so they are linked and never stored.
	ImageURL string
	Complete bool
}
