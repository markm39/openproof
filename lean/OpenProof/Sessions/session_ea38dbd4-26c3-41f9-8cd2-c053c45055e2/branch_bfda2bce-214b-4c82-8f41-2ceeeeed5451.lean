import Mathlib
/-!
OpenProof scratch session: Gcd Divides Left Factor Times k
Session id: session_ea38dbd4-26c3-41f9-8cd2-c053c45055e2
Mode: research
Updated: 
-/
-- branch: Gcd Divides Left Factor Times k repair [proving]
-- branch id: branch_bfda2bce-214b-4c82-8f41-2ceeeeed5451
-- node: Gcd Divides Left Factor Times k [proving]
-- kind: theorem
-- statement: theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides Left Factor Times k repair [pending]
-- artifact id: artifact_branch_bfda2bce-214b-4c82-8f41-2ceeeeed5451
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_bfda2bce-214b-4c82-8f41-2ceeeeed5451
-- label: Gcd Divides Left Factor Times k repair
-- metadata: {"hidden":true}

theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  simpa [Nat.mul_assoc] using congrArg (fun x => x * k) ht