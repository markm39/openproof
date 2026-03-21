import Mathlib
/-!
OpenProof scratch session: Fibonacci Gcd Identity Over `Nat`
Session id: session_15e1bdc3-f660-47fb-9279-52c93bae0ab7
Mode: research
Updated: 
-/
-- branch: Fibonacci Gcd Identity Over `Nat` planner [proving]
-- branch id: branch_48734f92-b436-47a5-b00b-87bcff151233
-- node: Fibonacci Gcd Identity Over `Nat` [proving]
-- kind: theorem
-- statement: `theorem fib_gcd (m n : ℕ) : Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n)`

-- artifact: Fibonacci Gcd Identity Over `Nat` planner [pending]
-- artifact id: artifact_branch_48734f92-b436-47a5-b00b-87bcff151233
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_48734f92-b436-47a5-b00b-87bcff151233
-- label: Fibonacci Gcd Identity Over `Nat` planner
-- metadata: {"hidden":true}

example
    (fib_gcd_mod_left :
      ∀ m n, Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.gcd (Nat.fib (m % n)) (Nat.fib n))
    {n : ℕ}
    (ih : ∀ k < n, ∀ t, Nat.gcd (Nat.fib t) (Nat.fib k) = Nat.fib (Nat.gcd t k))
    (hn : 0 < n) (m : ℕ) :
    Nat.gcd (Nat.fib m) (Nat.fib n) = Nat.fib (Nat.gcd m n) := by
  rw [fib_gcd_mod_left m n]
  rw [Nat.gcd_comm (Nat.fib (m % n)) (Nat.fib n)]
  rw [ih (m % n) (Nat.mod_lt _ hn) n]
  rw [Nat.gcd_rec m n]