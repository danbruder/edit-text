use oatie::doc::*;
use oatie::Operation;
use failure::Error;
use std::char::from_u32;
use super::walkers::*;

pub struct ActionContext {
    pub doc: Doc,
    pub client_id: String,
}

pub fn replace_block(ctx: ActionContext, tag: &str) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    walker.back_block();

    let len = if let Some(DocGroup(_, ref span)) = walker.doc.head() {
        span.skip_len()
    } else {
        println!("uhg {:?}", walker);
        unreachable!()
    };

    let mut writer = walker.to_writer();

    writer.del.group(&del_span![DelSkip(len)]);

    writer.add.group(
        &hashmap! { "tag".to_string() => tag.to_string() },
        &add_span![AddSkip(len)],
    );

    Ok(writer.result())
}

pub fn delete_char(ctx: ActionContext) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);

    // Check if we lead the block.
    let caret_pos = walker.caret_pos;
    let mut block_walker = walker.clone();
    block_walker.back_block();
    if caret_pos == block_walker.caret_pos {
        println!("TODO: merge blocks!");
        return Ok(op_span!([], []));
    }

    walker.back_char();

    // check if we are in a character group.
    if let Some(DocChars(..)) = walker.doc.head() {
        // fallthrough
    } else {
        return Ok(op_span!([], []));
    }

    let mut writer = walker.to_writer();

    writer.del.chars(1);
    writer.del.exit_all();

    writer.add.exit_all();

    Ok(writer.result())
}

pub fn add_char(ctx: ActionContext, key: u32) -> Result<Op, Error> {
    let mut writer = Walker::to_caret(&ctx.doc, &ctx.client_id).to_writer();

    writer.del.exit_all();

    // Insert new character.
    let c: char = from_u32(key).unwrap_or('?');
    writer.add.chars(&format!("{}", c));
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn split_block(ctx: ActionContext) -> Result<Op, Error> {
    let walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    let skip = walker.doc.skip_len();

    let previous_block = if let Some(DocGroup(attrs, _)) = walker.clone().back_block().doc.head() {
        attrs["tag"].to_string()
    } else {
        // Fill in default value.
        // TODO this should be a panic!
        "p".to_string()
    };

    let mut writer = walker.to_writer();

    writer.del.skip(skip);
    writer.del.close();
    writer.del.exit_all();

    writer
        .add
        .close(hashmap! { "tag".into() => previous_block });
    writer.add.begin();
    writer.add.skip(skip);
    writer
        .add
        .close(hashmap! { "tag".into() => "p".into() });
    writer.add.exit_all();

    Ok(writer.result())
}

pub fn caret_move(ctx: ActionContext, increase: bool) -> Result<Op, Error> {
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);

    // First operation removes the caret.
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op_1 = writer.result();

    // Second operation inserts the new caret.
    if increase {
        walker.next_char();
    } else {
        walker.back_char();
    }

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! { "tag".to_string() => "caret".to_string(), "client".to_string() => ctx.client_id.clone() });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    if increase {
        Ok(Operation::compose(&op_2, &op_1))
    } else {
        Ok(Operation::compose(&op_1, &op_2))
    }
}

pub fn cur_to_caret(ctx: ActionContext, cur: &CurSpan) -> Result<Op, Error> {
    // First operation removes the caret.
    let mut walker = Walker::to_caret(&ctx.doc, &ctx.client_id);
    let pos_1 = walker.caret_pos;
    let mut writer = walker.to_writer();

    writer.del.begin();
    writer.del.close();
    writer.del.exit_all();

    writer.add.exit_all();

    let op_1 = writer.result();

    // Second operation inserts a new caret.

    let mut walker = Walker::to_cursor(&ctx.doc, cur);
    let pos_2 = walker.caret_pos;
    if pos_1 == pos_2 {
        // Redundant
        return Ok(op_span!([], []));
    }

    let mut writer = walker.to_writer();

    writer.del.exit_all();

    writer.add.begin();
    writer.add.close(hashmap! { "tag".to_string() => "caret".to_string(), "client".to_string() => ctx.client_id.clone() });
    writer.add.exit_all();

    let op_2 = writer.result();

    // Return composed operations. Select proper order or otherwise composition
    // will be invalid.
    if pos_1 < pos_2 {
        Ok(Operation::compose(&op_2, &op_1))
    } else {
        Ok(Operation::compose(&op_1, &op_2))
    }
}
