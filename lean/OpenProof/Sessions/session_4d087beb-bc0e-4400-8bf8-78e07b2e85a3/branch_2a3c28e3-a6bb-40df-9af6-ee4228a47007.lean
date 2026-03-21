import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Multiple of The Left Argument
Session id: session_4d087beb-bc0e-4400-8bf8-78e07b2e85a3
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Multiple of The Left Argument prover [proving]
-- branch id: branch_2a3c28e3-a6bb-40df-9af6-ee4228a47007
-- node: Gcd Divides a Multiple of The Left Argument [proving]
-- kind: theorem
-- statement: theorem gcd_dvd_mul_right (m n k : Nat) : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides a Multiple of The Left Argument prover [pending]
-- artifact id: artifact_branch_2a3c28e3-a6bb-40df-9af6-ee4228a47007
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_2a3c28e3-a6bb-40df-9af6-ee4228a47007
-- label: Gcd Divides a Multiple of The Left Argument prover
-- metadata: {"hidden":false,"foreground":true}

theorem gcd_dvd_mul_right (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  exact dvd_mul_of_dvd_left (Nat.gcd_dvd_left m n) k