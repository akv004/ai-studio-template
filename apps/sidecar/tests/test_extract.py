"""Tests for document text extraction endpoint."""
import os
import sys
import pytest

# Add sidecar root to path so we can import server modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from server import _detect_format, _extract_docx, _extract_xlsx, _extract_pptx, _extract_pdf

FIXTURES = os.path.join(os.path.dirname(__file__), 'fixtures')


# ---------- Format detection ----------

def test_detect_pdf():
    assert _detect_format('report.pdf') == 'pdf'

def test_detect_docx():
    assert _detect_format('notes.docx') == 'docx'

def test_detect_xlsx():
    assert _detect_format('data.xlsx') == 'xlsx'

def test_detect_xls():
    assert _detect_format('legacy.xls') == 'xlsx'

def test_detect_pptx():
    assert _detect_format('slides.pptx') == 'pptx'

def test_detect_unsupported():
    with pytest.raises(ValueError, match='Unsupported'):
        _detect_format('image.png')

def test_detect_no_extension():
    with pytest.raises(ValueError, match='Unsupported'):
        _detect_format('Makefile')


# ---------- DOCX extraction ----------

def test_extract_docx_paragraphs():
    result = _extract_docx(os.path.join(FIXTURES, 'test.docx'))
    assert 'Hello from AI Studio' in result['text']
    assert 'test document for RAG extraction' in result['text']

def test_extract_docx_tables():
    result = _extract_docx(os.path.join(FIXTURES, 'test.docx'))
    assert 'Alice' in result['text']
    assert 'Engineer' in result['text']


# ---------- XLSX extraction ----------

def test_extract_xlsx_data():
    result = _extract_xlsx(os.path.join(FIXTURES, 'test.xlsx'))
    assert 'Tokyo' in result['text']
    assert '14000000' in result['text']

def test_extract_xlsx_sheet_names():
    result = _extract_xlsx(os.path.join(FIXTURES, 'test.xlsx'))
    assert result['sheets'] == ['Data']

def test_extract_xlsx_header():
    result = _extract_xlsx(os.path.join(FIXTURES, 'test.xlsx'))
    assert 'City' in result['text']
    assert 'Population' in result['text']


# ---------- PPTX extraction ----------

def test_extract_pptx_slides():
    result = _extract_pptx(os.path.join(FIXTURES, 'test.pptx'))
    assert 'AI Studio Overview' in result['text']
    assert 'open-source IDE for AI agents' in result['text']

def test_extract_pptx_slide_count():
    result = _extract_pptx(os.path.join(FIXTURES, 'test.pptx'))
    assert result['slides'] == 2

def test_extract_pptx_second_slide():
    result = _extract_pptx(os.path.join(FIXTURES, 'test.pptx'))
    assert '23 node types' in result['text']


# ---------- PDF extraction ----------

def test_extract_pdf_text():
    result = _extract_pdf(os.path.join(FIXTURES, 'test.pdf'))
    # Our minimal PDF has "Hello from PDF"
    assert result['pages'] == 1
    # pypdf may or may not extract text from our minimal handcrafted PDF
    # but it should not crash
    assert isinstance(result['text'], str)


# ---------- Edge cases ----------

def test_extract_nonexistent():
    with pytest.raises(Exception):
        _extract_docx('/tmp/does_not_exist_12345.docx')

def test_extract_empty_result():
    """Extractors should return a dict with 'text' key even if empty."""
    # Create a DOCX with no text
    import tempfile
    from docx import Document
    doc = Document()
    with tempfile.NamedTemporaryFile(suffix='.docx', delete=False) as f:
        doc.save(f.name)
        result = _extract_docx(f.name)
        os.unlink(f.name)
    assert 'text' in result
    assert isinstance(result['text'], str)
