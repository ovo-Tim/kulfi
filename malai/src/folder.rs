/// folder() exposes a folder on the kulfi network
///
/// the folder needs a little bit of user interface, the directory listing page. there are many
/// ways to implement the UI, we can hard code some minimal HTML template, and call it a day.
///
/// we are going to use fastn to build the UI though. partially, this is because we create the fastn
/// support in malai, which then will help us when we are building the kulfi app, which also uses
/// fastn internally for all sorts of UI.
///
/// using fastn means people can actually customize the folder browsing user interface, if we hard
/// code some HTML, we will have to make it configurable, and possibly use some sort of template
/// library. and if they want to do more, add logo, JS/css etc., it will no longer be just a single
/// html template, but we will need some way to include a folder, and we will end up either
/// re-inventing a poor man's web framework, or make this simple.
///
/// simple is in general good, but UI is a very important part of software, and giving it
/// second-rate treatment here, for folder, and not using fastn is a mistake. or so I feel as I
/// write this.
///
/// so how will this work? where would the fastn package be created? also which fastn template will
/// be used to create the fastn package?
///
/// at the highest level, as we have discussed in kulfi/src/config/mod.rs, we will have a kulfi
/// folder, which we will re-use for malai as well. why maintain two folders?
///
/// having said all that, the first version of malai browsing will be a simple HTML page, and we
/// will compile `folder.html` template as part of the build process.
pub async fn folder(_path: String, _graceful: kulfi_utils::Graceful) -> eyre::Result<()> {
    todo!()
}
