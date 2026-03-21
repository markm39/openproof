import Mathlib
/-!
OpenProof scratch session: Divisibility is Closed Under Addition
Session id: session_23198900-ecab-4271-a373-dc3dd0a221ec
Mode: research
Updated: 
-/
-- branch: Divisibility is Closed Under Addition prover [proving]
-- branch id: branch_d2f3afcb-df9c-4134-86a0-96f5f8adc9d5
-- node: Divisibility is Closed Under Addition [proving]
-- kind: theorem
-- statement: `theorem dvd_add_of_dvd {α : Type*} [Semiring α] {a b c : α} (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c)`

-- artifact: Divisibility is Closed Under Addition prover [pending]
-- artifact id: artifact_branch_d2f3afcb-df9c-4134-86a0-96f5f8adc9d5
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_d2f3afcb-df9c-4134-86a0-96f5f8adc9d5
-- label: Divisibility is Closed Under Addition prover
-- metadata: {"foreground":true}

theorem dvd_add_of_dvd {α : Type*} [Semiring α] {a b c : α}
    (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c) := by
  exact dvd_add hb hc