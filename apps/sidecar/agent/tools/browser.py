"""
Browser Tool
============
Browser automation using Playwright.
Navigate, screenshot, fill forms, extract data.
"""

import asyncio
import base64
from typing import Optional, Any
from dataclasses import dataclass, field


@dataclass
class BrowserResult:
    """Result of a browser action"""
    success: bool
    action: str
    data: Any = None
    screenshot: Optional[str] = None  # Base64 encoded
    error: Optional[str] = None


class BrowserTool:
    """
    Browser automation tool using Playwright.
    
    Example:
        browser = BrowserTool()
        await browser.start()
        
        result = await browser.navigate("https://example.com")
        result = await browser.screenshot()
        result = await browser.extract_text("h1")
        
        await browser.stop()
    """
    
    def __init__(
        self,
        headless: bool = True,
        timeout: float = 30000,  # ms
    ):
        self.headless = headless
        self.timeout = timeout
        self._playwright = None
        self._browser = None
        self._page = None
    
    async def start(self) -> BrowserResult:
        """Start the browser"""
        try:
            from playwright.async_api import async_playwright
            
            self._playwright = await async_playwright().start()
            self._browser = await self._playwright.chromium.launch(
                headless=self.headless,
            )
            self._page = await self._browser.new_page()
            self._page.set_default_timeout(self.timeout)
            
            return BrowserResult(success=True, action="start")
        
        except ImportError:
            return BrowserResult(
                success=False,
                action="start",
                error="Playwright not installed. Run: pip install playwright && playwright install chromium",
            )
        except Exception as e:
            return BrowserResult(success=False, action="start", error=str(e))
    
    async def stop(self) -> BrowserResult:
        """Stop the browser"""
        try:
            if self._browser:
                await self._browser.close()
            if self._playwright:
                await self._playwright.stop()
            
            self._page = None
            self._browser = None
            self._playwright = None
            
            return BrowserResult(success=True, action="stop")
        except Exception as e:
            return BrowserResult(success=False, action="stop", error=str(e))
    
    async def navigate(self, url: str) -> BrowserResult:
        """Navigate to a URL"""
        if not self._page:
            return BrowserResult(success=False, action="navigate", error="Browser not started")
        
        try:
            await self._page.goto(url, wait_until="domcontentloaded")
            return BrowserResult(
                success=True,
                action="navigate",
                data={"url": url, "title": await self._page.title()},
            )
        except Exception as e:
            return BrowserResult(success=False, action="navigate", error=str(e))
    
    async def screenshot(self, full_page: bool = False) -> BrowserResult:
        """Take a screenshot"""
        if not self._page:
            return BrowserResult(success=False, action="screenshot", error="Browser not started")
        
        try:
            screenshot_bytes = await self._page.screenshot(full_page=full_page)
            screenshot_b64 = base64.b64encode(screenshot_bytes).decode("utf-8")
            
            return BrowserResult(
                success=True,
                action="screenshot",
                screenshot=screenshot_b64,
            )
        except Exception as e:
            return BrowserResult(success=False, action="screenshot", error=str(e))
    
    async def click(self, selector: str) -> BrowserResult:
        """Click an element"""
        if not self._page:
            return BrowserResult(success=False, action="click", error="Browser not started")
        
        try:
            await self._page.click(selector)
            return BrowserResult(success=True, action="click", data={"selector": selector})
        except Exception as e:
            return BrowserResult(success=False, action="click", error=str(e))
    
    async def fill(self, selector: str, value: str) -> BrowserResult:
        """Fill a form field"""
        if not self._page:
            return BrowserResult(success=False, action="fill", error="Browser not started")
        
        try:
            await self._page.fill(selector, value)
            return BrowserResult(success=True, action="fill", data={"selector": selector})
        except Exception as e:
            return BrowserResult(success=False, action="fill", error=str(e))
    
    async def extract_text(self, selector: str = "body") -> BrowserResult:
        """Extract text content from element(s)"""
        if not self._page:
            return BrowserResult(success=False, action="extract_text", error="Browser not started")
        
        try:
            elements = await self._page.query_selector_all(selector)
            texts = []
            for el in elements:
                text = await el.inner_text()
                texts.append(text.strip())
            
            return BrowserResult(
                success=True,
                action="extract_text",
                data={"selector": selector, "texts": texts},
            )
        except Exception as e:
            return BrowserResult(success=False, action="extract_text", error=str(e))
    
    async def get_html(self, selector: str = "body") -> BrowserResult:
        """Get HTML content"""
        if not self._page:
            return BrowserResult(success=False, action="get_html", error="Browser not started")
        
        try:
            element = await self._page.query_selector(selector)
            if element:
                html = await element.inner_html()
                return BrowserResult(success=True, action="get_html", data={"html": html})
            return BrowserResult(success=False, action="get_html", error="Element not found")
        except Exception as e:
            return BrowserResult(success=False, action="get_html", error=str(e))
    
    async def evaluate(self, script: str) -> BrowserResult:
        """Execute JavaScript in the page"""
        if not self._page:
            return BrowserResult(success=False, action="evaluate", error="Browser not started")
        
        try:
            result = await self._page.evaluate(script)
            return BrowserResult(success=True, action="evaluate", data=result)
        except Exception as e:
            return BrowserResult(success=False, action="evaluate", error=str(e))
    
    async def wait_for(self, selector: str, timeout: Optional[float] = None) -> BrowserResult:
        """Wait for an element to appear"""
        if not self._page:
            return BrowserResult(success=False, action="wait_for", error="Browser not started")
        
        try:
            await self._page.wait_for_selector(selector, timeout=timeout)
            return BrowserResult(success=True, action="wait_for", data={"selector": selector})
        except Exception as e:
            return BrowserResult(success=False, action="wait_for", error=str(e))
