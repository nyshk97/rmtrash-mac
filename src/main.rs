use std::path::PathBuf;
use clap::Parser;
use anyhow::{Result, Context};
use core_foundation::url::CFURL;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use objc::{msg_send, sel, sel_impl, class, runtime::Object};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 再帰的に処理する（ディレクトリの場合）
    #[arg(short = 'r')]
    recursive: bool,

    /// 強制的に処理する（確認なし）
    #[arg(short = 'f')]
    force: bool,

    /// 処理対象のパス
    #[arg(required = true)]
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // パスが存在するか確認
    if !args.path.exists() {
        anyhow::bail!("パスが存在しません: {}", args.path.display());
    }

    // ディレクトリの場合の処理
    if args.path.is_dir() {
        if !args.recursive {
            anyhow::bail!("ディレクトリを移動するには `-r` オプションが必要です");
        }

        if !args.force {
            print!("ディレクトリ '{}' をゴミ箱に移動しますか？ [y/N] ", args.path.display());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("操作をキャンセルしました");
                return Ok(());
            }
        }
    }

    // ゴミ箱に移動
    move_to_trash(&args.path)?;
    println!("'{}' をゴミ箱に移動しました", args.path.display());

    Ok(())
}

fn move_to_trash(path: &PathBuf) -> Result<()> {
    let path_str = path.to_str()
        .context("パスをUTF-8文字列に変換できませんでした")?;

    let cf_path = CFString::new(path_str);
    let url = CFURL::from_file_system_path(
        cf_path,
        0, // kCFURLPOSIXPathStyle
        false,
    );

    // NSWorkspaceを使用してゴミ箱に移動
    unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let url_ref = url.as_concrete_TypeRef();
        let _: bool = msg_send![workspace, recycleURLs:&[url_ref] completionHandler:std::ptr::null_mut::<*mut Object>()];
    }

    Ok(())
}
