import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Multiple of The Left Argument
Session id: session_ba72f44a-9eb3-4e3e-bb47-1f8f2297406e
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Multiple of The Left Argument repair [proving]
-- branch id: branch_a76f3972-dd2c-432d-af59-6bb059feee6f
-- node: Gcd Divides a Multiple of The Left Argument [proving]
-- kind: theorem
-- statement: `theorem gcd_dvd_mul_left_factor (m n k : Nat) : Nat.gcd m n ∣ m * k`

-- artifact: Gcd Divides a Multiple of The Left Argument repair [pending]
-- artifact id: artifact_branch_a76f3972-dd2c-432d-af59-6bb059feee6f
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_a76f3972-dd2c-432d-af59-6bb059feee6f
-- label: Gcd Divides a Multiple of The Left Argument repair
-- metadata: {"hidden":true}

theorem gcd_dvd_mul_left_factor (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  conv_lhs => rw [ht]
  rw [Nat.mul_assoc]