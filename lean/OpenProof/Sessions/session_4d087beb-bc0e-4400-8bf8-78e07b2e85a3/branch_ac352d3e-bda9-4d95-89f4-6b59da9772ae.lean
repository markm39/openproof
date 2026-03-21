import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Multiple of The Left Argument
Session id: session_4d087beb-bc0e-4400-8bf8-78e07b2e85a3
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Multiple of The Left Argument planner [proving]
-- branch id: branch_ac352d3e-bda9-4d95-89f4-6b59da9772ae
-- node: Gcd Divides a Multiple of The Left Argument [verified]
-- kind: theorem
-- statement: theorem gcd_dvd_mul_right (m n k : Nat) : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides a Multiple of The Left Argument planner [pending]
-- artifact id: artifact_branch_ac352d3e-bda9-4d95-89f4-6b59da9772ae
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_ac352d3e-bda9-4d95-89f4-6b59da9772ae
-- label: Gcd Divides a Multiple of The Left Argument planner
-- metadata: {"hidden":true}

theorem gcd_dvd_mul_right (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  exact dvd_mul_of_dvd_left (Nat.gcd_dvd_left m n) k

-- fallback if the helper lemma name/order differs:
theorem gcd_dvd_mul_right' (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  calc
    m * k = (Nat.gcd m n * t) * k := by rw [ht]
    _ = Nat.gcd m n * (t * k) := by ac_rfl