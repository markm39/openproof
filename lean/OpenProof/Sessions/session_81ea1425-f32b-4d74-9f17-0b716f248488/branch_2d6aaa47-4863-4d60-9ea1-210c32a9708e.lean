import Mathlib
/-!
OpenProof scratch session: Nat Divisibility Under Addition
Session id: session_81ea1425-f32b-4d74-9f17-0b716f248488
Mode: research
Updated: 
-/
-- branch: Nat Divisibility Under Addition prover [proving]
-- branch id: branch_2d6aaa47-4863-4d60-9ea1-210c32a9708e
-- node: Nat Divisibility Under Addition [proving]
-- kind: theorem
-- statement: theorem nat_dvd_add (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c)

-- artifact: Nat Divisibility Under Addition prover [pending]
-- artifact id: artifact_branch_2d6aaa47-4863-4d60-9ea1-210c32a9708e
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_2d6aaa47-4863-4d60-9ea1-210c32a9708e
-- label: Nat Divisibility Under Addition prover
-- metadata: {"foreground":true}

theorem nat_dvd_add (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c) := by
  exact dvd_add hb hc