package fibonacci

type Fibonacci struct {
	cache map[int64]int64
}

func New() *Fibonacci {
	initial := make(map[int64]int64)
	initial[0] = 0
	initial[1] = 1

	return &Fibonacci{
		cache: initial,
	}
}

func (f Fibonacci) Get(n int64) int64 {
	if n < 0 {
		return 0
	}
	if n < 2 {
		return n
	}
	b2 := f.Get(n - 2)
	b1 := f.Get(n - 1)
	f.cache[n] = b1 + b2
	return b1 + b2
}
