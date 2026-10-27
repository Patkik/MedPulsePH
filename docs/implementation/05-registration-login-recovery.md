# 5. Registration, login, and recovery flows

**Status:** Planning requirements; implementation not started.

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I02

---

### Hospital onboarding

1. An authorized representative requests enrollment.
2. Platform operations verify the institution and representative.
3. The initial hospital administrator receives a time-limited invitation.
4. The administrator verifies the account and enrolls MFA.
5. The hospital configures departments, identity providers, and designated reviewers.
6. Required checks complete before the hospital becomes active.

An unrestricted public role selector must not create administrators.

### Staff onboarding

1. A hospital administrator invites a person for a proposed, scoped role.
2. The invitee establishes or links an account and verifies contact information.
3. The invitee enrolls required MFA.
4. Clinical staff submit credential evidence.
5. An independent reviewer approves eligibility.
6. The user sees only approved workspaces.

Account existence, hospital membership, and clinical eligibility are separate states. Editing an invitation URL cannot change a role. Invitations expire and require renewal rather than partial activation.

### Patient registration

1. Create an account and verify an available contact method.
2. Set up account authentication and provide minimum identity information.
3. Request linkage to an existing hospital record or start new enrollment.
4. Complete the hospital's identity-verification process.
5. Review emergency disclosure and set a six-digit clinical-authorization PIN through a patient-facing interaction.
6. Link and verify the band.

Matching demographic or contact details alone does not prove ownership of an existing record. Reception can prepare identity registration without a smartphone and must not see or retrieve the patient's PIN. Handle shared contact details, no email, similar names, spelling changes, lost bands, forgotten PINs, minors, and represented patients. A verified guardian relationship is separate from being an emergency contact.

Duplicate records require a dedicated review, evidence, scoped permission, independent approval where appropriate, and audit history; do not offer an informal merge shortcut.

### Login and recovery

Authenticate, perform required MFA, resolve hospital membership, select an authorized workspace if necessary, and open the permitted dashboard. Staff use the hospital identity provider where available. Account login and patient clinical-authorization PIN are distinct. Support password managers, autofill, and paste; use generic public login and recovery responses. Require reauthentication for sensitive actions. Do not automatically reopen a clinical chart after login.

Password recovery, MFA recovery, patient PIN reset, and membership restoration are separate flows. Staff-assisted recovery requires identity verification, audit, and notification through an established channel. See [OWASP authentication guidance](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html).
