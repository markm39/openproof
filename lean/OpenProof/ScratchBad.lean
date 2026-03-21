import Mathlib

theorem scratch_bad (a b c : Nat) (hb : a ∣ b) (hc : a ∣ c) : a ∣ (b + c) := by
  exact totally_fake_lemma hb hc
