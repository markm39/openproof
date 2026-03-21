import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Left Multiple
Session id: session_42882fe8-c568-4902-8e42-9aee38a05452
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Left Multiple prover [proving]
-- branch id: branch_ab94b02a-afcb-4935-bf37-ad0f56536ed9
-- node: Gcd Divides a Left Multiple [proving]
-- kind: theorem
-- statement: theorem gcd_dvd_mul (m n k : Nat) : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides a Left Multiple prover [pending]
-- artifact id: artifact_branch_ab94b02a-afcb-4935-bf37-ad0f56536ed9
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_ab94b02a-afcb-4935-bf37-ad0f56536ed9
-- label: Gcd Divides a Left Multiple prover
-- metadata: {"foreground":true}

theorem gcd_dvd_mul (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  exact dvd_mul_of_dvd_left (Nat.gcd_dvd_left m n) k