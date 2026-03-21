import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Multiple of The Left Argument
Session id: session_a0d3d04d-8b42-4373-8886-a82ad8252a61
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Multiple of The Left Argument prover [proving]
-- branch id: branch_3db9a454-45f6-48df-af91-5071ef3103f2
-- node: Gcd Divides a Multiple of The Left Argument [proving]
-- kind: theorem
-- statement: `theorem gcd_dvd_mul (m n k : Nat) : Nat.gcd m n ∣ m * k`

-- artifact: Gcd Divides a Multiple of The Left Argument prover [pending]
-- artifact id: artifact_branch_3db9a454-45f6-48df-af91-5071ef3103f2
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_3db9a454-45f6-48df-af91-5071ef3103f2
-- label: Gcd Divides a Multiple of The Left Argument prover
-- metadata: {"foreground":true}

theorem gcd_dvd_mul (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  exact dvd_mul_of_dvd_left (Nat.gcd_dvd_left m n) k