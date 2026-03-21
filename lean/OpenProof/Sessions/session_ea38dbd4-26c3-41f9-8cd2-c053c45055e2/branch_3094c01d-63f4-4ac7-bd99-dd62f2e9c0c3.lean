import Mathlib
/-!
OpenProof scratch session: Gcd Divides Left Factor Times k
Session id: session_ea38dbd4-26c3-41f9-8cd2-c053c45055e2
Mode: research
Updated: 
-/
-- branch: Gcd Divides Left Factor Times k prover [proving]
-- branch id: branch_3094c01d-63f4-4ac7-bd99-dd62f2e9c0c3
-- node: Gcd Divides Left Factor Times k [proving]
-- kind: theorem
-- statement: theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides Left Factor Times k prover [pending]
-- artifact id: artifact_branch_3094c01d-63f4-4ac7-bd99-dd62f2e9c0c3
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_3094c01d-63f4-4ac7-bd99-dd62f2e9c0c3
-- label: Gcd Divides Left Factor Times k prover
-- metadata: {"hidden":false,"foreground":true}

theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  rw [ht]
  simp [Nat.mul_assoc]