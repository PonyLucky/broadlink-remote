from typing import Dict, Any


def build_openapi() -> Dict[str, Any]:
    return {
        'openapi': '3.0.3',
        'info': {
            'title': 'Broadlink XML Controller API',
            'version': '1.0.0'
        },
        'paths': {
            '/doc/openapi.json': {
                'get': {'summary': 'Get OpenAPI document', 'responses': {'200': {'description': 'OK'}}}
            },
            '/controller': {
                'get': {'summary': 'List controllers', 'responses': {'200': {'description': 'OK'}}}
            },
            '/{c_name}/device': {
                'get': {
                    'summary': 'List devices of a controller',
                    'parameters': [{'name': 'c_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}}],
                    'responses': {'200': {'description': 'OK'}, '404': {'description': 'Not found'}}
                }
            },
            '/{c_name}/{d_name}': {
                'get': {
                    'summary': 'List commands and groups for a device',
                    'parameters': [
                        {'name': 'c_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                        {'name': 'd_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}}
                    ],
                    'responses': {'200': {'description': 'OK'}, '404': {'description': 'Not found'}}
                }
            },
            '/{c_name}/{d_name}/{cmd_name}': {
                'post': {
                    'summary': 'Send a command to a device',
                    'parameters': [
                        {'name': 'c_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                        {'name': 'd_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                        {'name': 'cmd_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                    ],
                    'responses': {
                        '200': {'description': 'OK'},
                        '400': {'description': 'Bad request'},
                        '403': {'description': 'Forbidden'},
                        '404': {'description': 'Not found'},
                        '502': {'description': 'Failed to send'}
                    }
                }
            }
        }
    }
