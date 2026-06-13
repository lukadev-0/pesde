//! Temporary script to generate JSON payloads for the registry,
//! because the CLI hasn't been updated yet.

use std::path::Path;

use pesde::signature;
use pesde::signature::PublicKey;
use pesde::signature::Signature;
use pesde::source::pesde::registry::IdentityId;
use pesde::source::pesde::registry::PublishBody;
use pesde::source::pesde::registry::RegisterIdentityBody;
use pesde::source::pesde::registry::ScopeEntryBody;
use pesde::source::pesde::registry::ScopeManifestUpdateBody;
use pesde::source::pesde::registry::SignedEntry;
use pesde::source::pesde::registry::canonical_bytes;
use serde::Serialize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let registry_fixtures = Path::new(file!())
		.parent()
		.unwrap()
		.join("../../crates/registry/fixtures")
		.canonicalize()?;

	let identity_id = IdentityId("019ec2c7-ab12-7c03-bc15-e55adaa98bf1".parse()?);
	let pub_key = ssh_key::PublicKey::read_openssh_file(&registry_fixtures.join("key.pub"))?;
	let priv_key = ssh_key::PrivateKey::read_openssh_file(&registry_fixtures.join("key"))?;

	print_signed_entry(
		&priv_key,
		&RegisterIdentityBody {
			identity_id,
			public_key: PublicKey::new(
				signature::KeyKind::Ed25519,
				pub_key.key_data().ed25519().unwrap().0,
			)
			.unwrap(),
		},
	);

	print_signed_entry(&priv_key, &ScopeEntryBody {
		scope: "test".parse().unwrap(),
		author_identity: identity_id,
		payload: PublishBody {
			name: "package".parse().unwrap(),
			version: "1.0.0".parse().unwrap(),
			archive_hash: "sha384:sdrqzhcxy9ace9x33zmcwqycws3n381dtzw5me2xjs9qh8y91vc85x2c673mc180e5szxbtc8kg8c"
				.parse()
				.unwrap(),
			description: Default::default(),
			license: Default::default(),
			authors: Default::default(),
			repository: None,
			dependencies: Default::default(),
		},
	});

	print_signed_entry(
		&priv_key,
		&ScopeEntryBody {
			scope: "test".parse().unwrap(),
			author_identity: identity_id,
			payload: ScopeManifestUpdateBody {
				manifest: pesde::source::pesde::registry::ScopeManifest {
					owner: identity_id,
					members: Default::default(),
				},
			},
		},
	);

	Ok(())
}

fn print_signed_entry<T: Serialize>(priv_key: &ssh_key::PrivateKey, value: &T) {
	let canon = canonical_bytes(&value);
	let sig = priv_key
		.sign("pesde signature", ssh_key::HashAlg::Sha512, &canon)
		.unwrap();
	let sig = Signature::new(
		signature::SignatureKind::SshEd25519Sha512,
		sig.signature_bytes(),
	)
	.unwrap();
	let entry = SignedEntry::new(sig, value);
	serde_json::to_writer(std::io::stdout(), &entry).unwrap();
	println!();
}
