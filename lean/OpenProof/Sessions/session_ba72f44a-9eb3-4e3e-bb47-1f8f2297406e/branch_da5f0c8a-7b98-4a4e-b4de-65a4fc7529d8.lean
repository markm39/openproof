import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Multiple of The Left Argument
Session id: session_ba72f44a-9eb3-4e3e-bb47-1f8f2297406e
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Multiple of The Left Argument planner [proving]
-- branch id: branch_da5f0c8a-7b98-4a4e-b4de-65a4fc7529d8
-- node: Gcd Divides a Multiple of The Left Argument [proving]
-- kind: theorem
-- statement: `theorem gcd_dvd_mul_left_factor (m n k : Nat) : Nat.gcd m n ∣ m * k`

-- artifact: Gcd Divides a Multiple of The Left Argument planner [pending]
-- artifact id: artifact_branch_da5f0c8a-7b98-4a4e-b4de-65a4fc7529d8
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_da5f0c8a-7b98-4a4e-b4de-65a4fc7529d8
-- label: Gcd Divides a Multiple of The Left Argument planner
-- metadata: {"hidden":true}

theorem gcd_dvd_mul_left_factor (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  calc
    m * k = (Nat.gcd m n * t) * k := by rw [ht]
    _ = Nat.gcd m n * (t * k) := by rw [Nat.mul_assoc]