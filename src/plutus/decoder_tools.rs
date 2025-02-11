use pallas_primitives::conway::{Constr, PlutusData};
use serde_json::{json, Value};
use uplc::ast::{Constant, Program, Term, Type, NamedDeBruijn};
use blst::*;

/// Converts a UPLC program into its JSON representation.
///
/// # Arguments
///
/// * `program` - A reference to a UPLC Program (using NamedDeBruijn).
///
/// # Returns
///
/// A JSON string representing the program.
pub fn to_json_program(program: &Program<NamedDeBruijn>) -> Value {
    let version = format!("{}.{}.{}", program.version.0, program.version.1, program.version.2);
    json!({
        "program": {
            "version": version,
            "term": to_json_term(&program.term)
        }
    })
}

/// Recursively converts a UPLC term into a JSON value.
///
/// # Arguments
///
/// * `term` - A reference to a UPLC Term.
///
/// # Returns
///
/// A serde_json::Value representing the term.
pub fn to_json_term(term: &Term<NamedDeBruijn>) -> Value {
    match term {
        Term::Var(name) => json!({ "var": name.text }),
        Term::Delay(inner) => json!({ "delay": to_json_term(inner) }),
        Term::Lambda { parameter_name, body } => json!({
            "lambda": {
                "parameter_name": parameter_name.text,
                "body": to_json_term(body)
            }
        }),
        Term::Apply { function, argument } => json!({
            "apply": {
                "function": to_json_term(function),
                "argument": to_json_term(argument)
            }
        }),
        Term::Constant(constant) => json!({ "constant": to_json_constant(constant) }),
        Term::Force(inner) => json!({ "force": to_json_term(inner) }),
        Term::Error => json!({ "error": "error" }),
        Term::Builtin(builtin) => json!({ "builtin": builtin.to_string() }),
        Term::Constr { tag, fields } => json!({
            "constr": {
                "tag": tag,
                "fields": fields.iter().map(to_json_term).collect::<Vec<Value>>()
            }
        }),
        Term::Case { constr, branches } => json!({
            "case": {
                "constr": to_json_term(constr),
                "branches": branches.iter().map(to_json_term).collect::<Vec<Value>>()
            }
        }),
    }
}

/// Converts a UPLC constant into its JSON representation.
fn to_json_constant(constant: &Constant) -> Value {
    match constant {
        Constant::Integer(i) => json!({ "integer": i.to_string() }),
        Constant::ByteString(bs) => json!({ "bytestring": hex::encode(bs) }),
        Constant::String(s) => json!({ "string": s }),
        Constant::Unit => json!({ "unit": "()" }),
        Constant::Bool(b) => json!({ "bool": b }),
        Constant::ProtoList(ty, items) => json!({
            "list": {
                "type": to_json_type(ty),
                "items": items.iter().map(to_json_constant).collect::<Vec<Value>>()
            }
        }),
        Constant::ProtoPair(left_type, right_type, left, right) => json!({
            "pair": {
                "type_left": to_json_type(left_type),
                "type_right": to_json_type(right_type),
                "left": to_json_constant(left),
                "right": to_json_constant(right)
            }
        }),
        Constant::Data(d) => json!({ "data": to_json_plutus_data(d) }),
        Constant::Bls12_381G1Element(p1) => json!({
            "bls12_381_G1_element": {
                "x": p1.x.l,
                "y": p1.y.l,
                "z": p1.z.l
            }
        }),
        Constant::Bls12_381G2Element(p2) => json!({
            "bls12_381_G2_element": json_blst_p2(p2)
        }),
        Constant::Bls12_381MlResult(_) => {
            panic!("Cannot represent Bls12_381MlResult as json")
        }
    }
}

/// Converts a blst_p2 element into JSON.
fn json_blst_p2(p2: &blst_p2) -> Value {
    json!({
        "x": to_json_blst_fp2(&p2.x),
        "y": to_json_blst_fp2(&p2.y),
        "z": to_json_blst_fp2(&p2.z),
    })
}

/// Converts a blst_fp2 element into a JSON array.
fn to_json_blst_fp2(fp2: &blst_fp2) -> Value {
    Value::Array(fp2.fp.iter().map(|fp| json!(fp.l)).collect())
}

/// Converts Plutus data into its JSON representation.
fn to_json_plutus_data(data: &PlutusData) -> Value {
    match data {
        PlutusData::Constr(Constr { tag, any_constructor, fields }) => json!({
            "constr": {
                "tag": tag,
                "any_constructor": any_constructor,
                "fields": fields.iter().map(to_json_plutus_data).collect::<Vec<Value>>()
            }
        }),
        PlutusData::Map(kvp) => json!({
            "map": kvp.iter().map(|(key, value)| {
                json!({
                    "key": to_json_plutus_data(key),
                    "value": to_json_plutus_data(value)
                })
            }).collect::<Vec<Value>>()
        }),
        PlutusData::BigInt(bi) => json!({ "integer": bi }),
        PlutusData::BoundedBytes(bs) => json!({ "bytestring": hex::encode(bs.to_vec()) }),
        PlutusData::Array(a) => json!({
            "list": a.iter().map(to_json_plutus_data).collect::<Vec<Value>>()
        }),
    }
}

/// Converts a UPLC type into its JSON representation.
fn to_json_type(term_type: &Type) -> Value {
    match term_type {
        Type::Bool => json!("bool"),
        Type::Integer => json!("integer"),
        Type::String => json!("string"),
        Type::ByteString => json!("bytestring"),
        Type::Unit => json!("unit"),
        Type::List(ty) => json!({
            "list": to_json_type(ty)
        }),
        Type::Pair(left, right) => json!({
            "pair": {
                "left": to_json_type(left),
                "right": to_json_type(right)
            }
        }),
        Type::Data => json!("data"),
        Type::Bls12_381G1Element => json!("bls12_381_G1_element"),
        Type::Bls12_381G2Element => json!("bls12_381_G2_element"),
        Type::Bls12_381MlResult => json!("bls12_381_mlresult"),
    }
}