"""Compatibility entry point for the current approved form-layer verification."""
from pathlib import Path
import runpy
runpy.run_path(str(Path(__file__).with_name('verify-creative-art.py')),run_name='__main__')
