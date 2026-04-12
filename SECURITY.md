# Security Policy

## Current Status

FlowFi Protocol is in active development and **has not been audited by an external security firm**. The contracts are deployed on Stellar Testnet only. Do not use with real funds on mainnet.

## Scope

The following are in scope for security reports:

- Smart contract vulnerabilities (`contracts/`)
- SDK issues that could lead to loss of funds or unauthorized access (`sdk/`)
- Deployment script issues that could expose private keys

## Out of Scope

- Issues in third-party dependencies (report to the relevant maintainer)
- UI issues without on-chain impact
- Theoretical attacks with no practical exploit path

## Reporting a Vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Email: security@flowfi.xyz

Include:
1. A clear description of the vulnerability
2. Steps to reproduce (proof of concept preferred)
3. Your assessment of the severity and impact
4. Any suggested fixes

We will acknowledge receipt within 48 hours and aim to respond substantively within 5 business days.

## Known Limitations

The following are known security limitations that are tracked as open issues:

- No emergency pause mechanism ([Issue #18](../../issues/18))
- Reward rate changes don't checkpoint all users atomically ([Issue #19](../../issues/19))
- Share price inflation attack surface on first deposit ([Issue #11](../../issues/11))
- No reentrancy analysis has been performed ([Issue #20](../../issues/20))
- Admin is a single EOA with no timelock — no governance yet ([Issue #17](../../issues/17))

## Disclosure Policy

We follow responsible disclosure. We ask reporters to give us 90 days to address a vulnerability before public disclosure. We will credit reporters who follow this policy in our changelog.
