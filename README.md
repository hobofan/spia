## Motivation

The goal of this project is to create a dataset of annotated regions in images of scientific papers that have been rendered (Render scientific paper (PDF) to multiple images -> annotate regions in those images).

The desired use-case for this dataset is as training data for a machine learning algorithm that can automatically segment an input image of a page of a scientific paper into regions. In a next step those regions should be usable as input for a OCR program. The hope is that is by pre-segmenting the text regions, OCR quality can be improved and a semantic stucture can be given to the stitched output text.

## Annotation file format (V1)

```json
{
  "version": "1",
  "annotations": [
    {
      "subject": {
        "id_doi": "",
        "download_url": "www.semantic-web-journal.net/sites/default/files/swj120_2.pdf",
        "download_checksum_sha256": "6c8716e634199f4cde899f84f7de22fd518d8f5996ad1f7b0e66b31a93efc88f"
      },
      "pages": [
        {
          "document_page_number": 1,
          "height": 2376,
          "width": 1836,
          "regions": [
            {
              "region_shape": "rect",
              "x1": 10,
              "y1": 10,
              "x2": 20,
              "y2": 20,
              "region_attributes": {
                "type": "reference_item"
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### Region Attributes

#### type

Possible values:

- `reference_item`: A single reference item in the reference section (usually found at the end of a paper). The region should contain both the reference id and the reference text.
- `reference_item_id`: The id of a reference item in the reference section. Most often a number, but may differ by citation style (e.g. `Smith2007`). The region should contain brackets and/or a colon or other seperators between the id and the reference text following it.
- `reference_item_text`: The text of a reference item that comprises the reference information. Usually contains authors, year of publication, journal, etc..
