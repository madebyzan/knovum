-- Recognized text belongs to the picture it was read from. The first reader kept a copy of all of
-- a node's text in its textRecognition record, read an image's original file instead of what it
-- shows, and never read video covers; its passages say no more than which node they came from, so
-- nothing can tell which picture of a document they belong to. They cannot be carried forward, so
-- every node is read again: the records are withdrawn, and so are the jobs that answered them, which
-- otherwise mark a document as already read at its unchanged revision. The startup sweep queues the
-- new reading; publishing it replaces the old recognized passages and detaches what cited them, so
-- the text stays findable until then.
UPDATE spaces_nodes
  SET data = json_remove(data, '$.textRecognition')
  WHERE json_extract(data, '$.textRecognition') IS NOT NULL;

DELETE FROM spaces_enrichment_jobs WHERE job_type = 'recognize_text';
