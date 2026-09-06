package domain

// Macros are the four figures the user plans against, per 100 units for a product
// and in total elsewhere.
type Macros struct {
	EnergyKcal     float64
	FatG           float64
	ProteinG       float64
	CarbohydratesG float64
}

func (m Macros) Add(other Macros) Macros {
	return Macros{
		EnergyKcal:     m.EnergyKcal + other.EnergyKcal,
		FatG:           m.FatG + other.FatG,
		ProteinG:       m.ProteinG + other.ProteinG,
		CarbohydratesG: m.CarbohydratesG + other.CarbohydratesG,
	}
}

func (m Macros) Scale(factor float64) Macros {
	return Macros{
		EnergyKcal:     m.EnergyKcal * factor,
		FatG:           m.FatG * factor,
		ProteinG:       m.ProteinG * factor,
		CarbohydratesG: m.CarbohydratesG * factor,
	}
}
