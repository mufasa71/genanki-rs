import os
import anki
import anki.importing.apkg
import anki.collection
import anki.lang


anki.lang.set_lang("en_GB")


def setup(fname):
    colf_name = f"{fname}.anki2"
    return anki.collection.Collection(colf_name)


def check_media(col):
    # col.media.check seems to assume that the cwd is the media directory. So this helper function
    # chdirs to the media dir before running check and then goes back to the original cwd.
    orig_cwd = os.getcwd()
    os.chdir(col.media.dir())
    res = col.media.check()
    os.chdir(orig_cwd)
    return res.missing, res.report, res.unused


# outfile = "/home/smk/Sync/media_files.anki"
# col = setup("test_setup")
# importer = anki.importing.apkg.AnkiPackageImporter(col, outfile)
# importer.run()
# res = col
#
# result = check_media(col)
#
# print(col.media.check())
