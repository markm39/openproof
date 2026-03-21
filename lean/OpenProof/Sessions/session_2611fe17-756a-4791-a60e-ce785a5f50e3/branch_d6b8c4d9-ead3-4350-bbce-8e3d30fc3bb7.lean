import Mathlib
/-!
OpenProof scratch session: Nat.gcd Divides The Left Argument
Session id: session_2611fe17-756a-4791-a60e-ce785a5f50e3
Mode: research
Updated: 
-/
-- branch: Nat.gcd Divides The Left Argument prover [proving]
-- branch id: branch_d6b8c4d9-ead3-4350-bbce-8e3d30fc3bb7
-- node: Nat.gcd Divides The Left Argument [proving]
-- kind: theorem
-- statement: theorem gcd_dvd_left_target (m n : Nat) : Nat.gcd m n ∣ m

-- artifact: Nat.gcd Divides The Left Argument prover [pending]
-- artifact id: artifact_branch_d6b8c4d9-ead3-4350-bbce-8e3d30fc3bb7
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_d6b8c4d9-ead3-4350-bbce-8e3d30fc3bb7
-- label: Nat.gcd Divides The Left Argument prover
-- metadata: {"foreground":true}

theorem gcd_dvd_left_target (m n : Nat) : Nat.gcd m n ∣ m := by
  exact Nat.gcd_dvd_left m n