import Mathlib
/-!
OpenProof scratch session: Nat Divisibility Under Addition
Session id: session_0a480dce-6aa6-4cbe-acec-cfcd196c79a2
Mode: research
Updated: 
-/
-- branch: Nat Divisibility Under Addition prover [proving]
-- branch id: branch_170c023f-cf02-48a4-9c07-735c70577068
-- node: Nat Divisibility Under Addition [proving]
-- kind: theorem
-- statement: theorem gcd_dvd_left_target (m n : Nat) : Nat.gcd m n ∣ m

-- artifact: Nat Divisibility Under Addition prover [pending]
-- artifact id: artifact_branch_170c023f-cf02-48a4-9c07-735c70577068
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_170c023f-cf02-48a4-9c07-735c70577068
-- label: Nat Divisibility Under Addition prover
-- metadata: {"foreground":true}

theorem gcd_dvd_left_target (m n : Nat) : Nat.gcd m n ∣ m := by
  exact Nat.gcd_dvd_left m n