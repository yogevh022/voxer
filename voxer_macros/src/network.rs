extern crate proc_macro;

use crate::common::AttributeArgs;
use quote;
use syn;
use syn::Expr;
use syn::parse::{Parse, Parser};

#[derive(Debug)]
pub(crate) enum FragmentCount {
    Auto, // todo auto frag count logic
    Fixed(Expr),
}

#[derive(Debug)]
pub(crate) struct NetworkMessageConfig {
    pub(crate) tag: Expr,
    pub(crate) fragment_count: FragmentCount,
}

pub(crate) fn parse_network_message_config(
    attr_args: AttributeArgs,
) -> Result<NetworkMessageConfig, &'static str> {
    let mut tag_val = None;
    let mut frag_count_val = FragmentCount::Auto;

    for arg in attr_args.0 {
        match arg.key.to_string().as_str() {
            "tag" => tag_val = Some(arg.value),
            "frags" => frag_count_val = FragmentCount::Fixed(arg.value),
            _ => {},
        }
    }

    Ok(NetworkMessageConfig {
        tag: tag_val.unwrap(),
        fragment_count: frag_count_val,
    })
}
