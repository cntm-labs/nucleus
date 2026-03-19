from django.conf import settings
from .verify import verify_token

class NucleusMiddleware:
    def __init__(self, get_response):
        self.get_response = get_response
        self.base_url = getattr(settings, 'NUCLEUS_BASE_URL', 'https://api.nucleus.dev')

    def __call__(self, request):
        auth = request.META.get('HTTP_AUTHORIZATION', '')
        if auth.startswith('Bearer '):
            try:
                request.nucleus_claims = verify_token(auth[7:], self.base_url)
            except Exception:
                request.nucleus_claims = None
        else:
            request.nucleus_claims = None
        return self.get_response(request)

def nucleus_claims(request):
    return getattr(request, 'nucleus_claims', None)
