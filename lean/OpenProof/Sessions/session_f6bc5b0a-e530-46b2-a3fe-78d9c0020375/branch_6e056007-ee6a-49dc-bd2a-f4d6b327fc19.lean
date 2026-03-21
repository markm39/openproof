import Mathlib
/-!
OpenProof scratch session: Nat.gcd Divides The Left Argument
Session id: session_f6bc5b0a-e530-46b2-a3fe-78d9c0020375
Mode: research
Updated: 
-/
-- branch: Nat.gcd Divides The Left Argument prover [proving]
-- branch id: branch_6e056007-ee6a-49dc-bd2a-f4d6b327fc19
-- node: Nat.gcd Divides The Left Argument [proving]
-- kind: theorem
-- statement: `theorem gcd_dvd_left_nat (m n : Nat) : Nat.gcd m n ∣ m`

-- artifact: Nat.gcd Divides The Left Argument prover [pending]
-- artifact id: artifact_branch_6e056007-ee6a-49dc-bd2a-f4d6b327fc19
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_6e056007-ee6a-49dc-bd2a-f4d6b327fc19
-- label: Nat.gcd Divides The Left Argument prover
-- metadata: {"foreground":true}

theorem gcd_dvd_left_nat (m n : Nat) : Nat.gcd m n ∣ m := by
  exact Nat.gcd_dvd_left m n