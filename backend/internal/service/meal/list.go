package meal

func (s *Service) List() []Meal {
	s.mu.RLock()
	defer s.mu.RUnlock()

	return append([]Meal(nil), s.meals...)
}
