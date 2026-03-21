import Mathlib
/-!
OpenProof scratch session: Nat Divisibility Under Addition
Session id: session_d22e95c6-6740-4abe-b65d-e82ee9d8f4c2
Mode: research
Updated: 
-/
-- branch: Nat Divisibility Under Addition prover [proving]
-- branch id: branch_84ae9f66-15bc-4ea0-b63c-cab914f63685
-- node: Nat Divisibility Under Addition [proving]
-- kind: theorem
-- statement: theorem nat_dvd_add (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c)

-- artifact: Nat Divisibility Under Addition prover [pending]
-- artifact id: artifact_branch_84ae9f66-15bc-4ea0-b63c-cab914f63685
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_84ae9f66-15bc-4ea0-b63c-cab914f63685
-- label: Nat Divisibility Under Addition prover
-- metadata: {"foreground":true}

theorem nat_dvd_add (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c) := by
  exact dvd_add_wrong hb hc