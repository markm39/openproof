import Mathlib
/-!
OpenProof scratch session: Gcd Divides Left Factor Times k
Session id: session_f54fd86e-e870-4b52-b660-2d3a3ad3b616
Mode: research
Updated: 
-/
-- branch: Gcd Divides Left Factor Times k prover [proving]
-- branch id: branch_02f77191-6c87-4f8f-93f0-f75e833dc11f
-- node: Gcd Divides Left Factor Times k [proving]
-- kind: theorem
-- statement: theorem gcd_dvd_mul (m n k : ℕ) : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides Left Factor Times k prover [pending]
-- artifact id: artifact_branch_02f77191-6c87-4f8f-93f0-f75e833dc11f
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_02f77191-6c87-4f8f-93f0-f75e833dc11f
-- label: Gcd Divides Left Factor Times k prover
-- metadata: {"hidden":false,"foreground":true}

theorem gcd_dvd_mul (m n k : ℕ) : Nat.gcd m n ∣ m * k := by
  exact dvd_mul_of_dvd_left (Nat.gcd_dvd_left m n) k