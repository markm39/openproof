import Mathlib
/-!
OpenProof scratch session: Nat Divisibility Under Addition
Session id: session_b3eb451a-c31c-4cd6-99bc-0e4094c8a3a8
Mode: research
Updated: 
-/
-- branch: Nat Divisibility Under Addition prover [proving]
-- branch id: branch_b9061019-39e1-4110-b2bd-4cb224b3e97e
-- node: Nat Divisibility Under Addition [proving]
-- kind: theorem
-- statement: theorem gcd_dvd_left_target (m n : Nat) : Nat.gcd m n ∣ m

-- artifact: Nat Divisibility Under Addition prover [pending]
-- artifact id: artifact_branch_b9061019-39e1-4110-b2bd-4cb224b3e97e
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_b9061019-39e1-4110-b2bd-4cb224b3e97e
-- label: Nat Divisibility Under Addition prover
-- metadata: {"foreground":true}

theorem gcd_dvd_left_target (m n : Nat) : Nat.gcd m n ∣ m := by
  exact Nat.gcd_dvd_left m n