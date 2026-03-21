import Mathlib
/-!
OpenProof scratch session: Nat Divisibility Under Addition
Session id: session_375702b4-7ab7-4617-adca-40da4fa67349
Mode: research
Updated: 
-/
-- branch: Nat Divisibility Under Addition prover [proving]
-- branch id: branch_b88dd2da-f9f6-4136-a2dd-3b1f4a75a7c0
-- node: Nat Divisibility Under Addition [proving]
-- kind: theorem
-- statement: theorem nat_dvd_add (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c)

-- artifact: Nat Divisibility Under Addition prover [pending]
-- artifact id: artifact_branch_b88dd2da-f9f6-4136-a2dd-3b1f4a75a7c0
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_b88dd2da-f9f6-4136-a2dd-3b1f4a75a7c0
-- label: Nat Divisibility Under Addition prover
-- metadata: {"foreground":true}

theorem nat_dvd_add (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c) := by
  rcases hb with ⟨m, rfl⟩
  rcases hc with ⟨n, rfl⟩
  refine ⟨m + n, ?_⟩
  simpa [Nat.mul_add]